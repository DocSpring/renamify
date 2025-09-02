import type { ChildProcess } from 'node:child_process';
import { spawn } from 'node:child_process';
import * as fs from 'node:fs';
import * as path from 'node:path';
import * as vscode from 'vscode';

type RequestState =
  | 'pending'
  | 'processing'
  | 'spawning'
  | 'running'
  | 'completed'
  | 'cancelled';

type Request = {
  id: number;
  args: string[];
  cliPath: string;
  resolve: (value: string | null) => void;
  reject: (error: Error) => void;
  state: RequestState;
  timestamp: number;
};

/**
 * ProcessCoordinator ensures only one CLI process runs at a time
 * It handles the ENTIRE lifecycle: queue management, process spawning, and result handling
 */
export class ProcessCoordinator {
  private currentProcess: ChildProcess | null = null;
  private activeRequest: Request | null = null;
  private pendingRequest: Request | null = null;
  private requestCounter = 0;
  private readonly instanceId: string;
  private isKilling = false;

  constructor(
    private readonly getLockFilePath: () => string = () => {
      const workspaceRoot =
        vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || process.cwd();
      return path.join(workspaceRoot, '.renamify', 'renamify.lock');
    }
  ) {
    this.instanceId = `PC-${Math.random().toString(36).substr(2, 9)}`;
    console.log(
      `[${new Date().toISOString()}] [ProcessCoordinator ${this.instanceId}] CONSTRUCTOR`
    );
  }

  /**
   * Execute a CLI command with proper serialization
   * Returns a promise that resolves with the command output
   */
  execute(cliPath: string, args: string[]): Promise<string | null> {
    const requestId = ++this.requestCounter;
    const request: Request = {
      id: requestId,
      args,
      cliPath,
      resolve: () => {}, // Will be set below
      reject: () => {}, // Will be set below
      state: 'pending',
      timestamp: Date.now(),
    };

    console.log(
      `[${new Date().toISOString()}] [ProcessCoordinator ${this.instanceId}] [#${requestId}] EXECUTE REQUESTED: ${args.join(' ')}`
    );

    return new Promise<string | null>((resolve, reject) => {
      request.resolve = resolve;
      request.reject = reject;

      // Cancel any existing pending request
      if (this.pendingRequest) {
        console.log(
          `[${new Date().toISOString()}] [ProcessCoordinator ${this.instanceId}] [#${requestId}] CANCELLING PENDING REQUEST #${this.pendingRequest.id}`
        );
        this.pendingRequest.state = 'cancelled';
        this.pendingRequest.resolve(null); // Return null for cancelled requests
        this.pendingRequest = null;
      }

      if (this.activeRequest) {
        // Store as new pending request
        this.pendingRequest = request;
        console.log(
          `[${new Date().toISOString()}] [ProcessCoordinator ${this.instanceId}] [#${requestId}] STORED AS PENDING (active: #${this.activeRequest.id})`
        );
      } else {
        // Process immediately
        this.activeRequest = request;
        this.activeRequest.state = 'processing';
        console.log(
          `[${new Date().toISOString()}] [ProcessCoordinator ${this.instanceId}] [#${requestId}] PROCESSING IMMEDIATELY`
        );

        // Use setImmediate to ensure proper async execution
        setImmediate(() => this.processActiveRequest());
      }
    });
  }

  private async processActiveRequest(): Promise<void> {
    if (!this.activeRequest) {
      console.error(
        `[ProcessCoordinator ${this.instanceId}] processActiveRequest called with no active request!`
      );
      return;
    }

    const request = this.activeRequest;
    console.log(
      `[${new Date().toISOString()}] [ProcessCoordinator ${this.instanceId}] [#${request.id}] STARTING PROCESSING`
    );

    try {
      // Kill any existing process first
      if (this.currentProcess) {
        console.log(
          `[${new Date().toISOString()}] [ProcessCoordinator ${this.instanceId}] [#${request.id}] KILLING EXISTING PROCESS PID=${this.currentProcess.pid}`
        );
        await this.killCurrentProcess(request.id);
      }

      // Wait for lock file to be cleared
      await this.waitForLockFile(request.id);

      // Now spawn the new process
      request.state = 'spawning';
      console.log(
        `[${new Date().toISOString()}] [ProcessCoordinator ${this.instanceId}] [#${request.id}] SPAWNING PROCESS: ${request.cliPath} ${request.args.join(' ')}`
      );

      const result = await this.spawnAndWait(request);

      request.state = 'completed';
      console.log(
        `[${new Date().toISOString()}] [ProcessCoordinator ${this.instanceId}] [#${request.id}] COMPLETED SUCCESSFULLY`
      );
      request.resolve(result);
    } catch (error) {
      console.error(
        `[${new Date().toISOString()}] [ProcessCoordinator ${this.instanceId}] [#${request.id}] FAILED: ${error}`
      );
      request.state = 'cancelled';

      // If the command was cancelled (we killed it), resolve with null instead of rejecting
      if (error instanceof Error && error.message === 'Command was cancelled') {
        request.resolve(null);
      } else {
        request.reject(
          error instanceof Error ? error : new Error(String(error))
        );
      }
    }

    // Clear active request
    this.activeRequest = null;

    // Process pending request if any
    if (this.pendingRequest) {
      console.log(
        `[${new Date().toISOString()}] [ProcessCoordinator ${this.instanceId}] PROCESSING PENDING REQUEST #${this.pendingRequest.id}`
      );
      this.activeRequest = this.pendingRequest;
      this.activeRequest.state = 'processing';
      this.pendingRequest = null;

      // Process asynchronously
      setImmediate(() => this.processActiveRequest());
    } else {
      console.log(
        `[${new Date().toISOString()}] [ProcessCoordinator ${this.instanceId}] NO PENDING REQUESTS - IDLE`
      );
    }
  }

  private spawnAndWait(request: Request): Promise<string> {
    return new Promise((resolve, reject) => {
      const workspaceRoot =
        vscode.workspace.workspaceFolders?.[0]?.uri.fsPath || process.cwd();

      console.log(
        `[ProcessCoordinator ${this.instanceId}] [#${request.id}] Spawning: ${request.cliPath} ${request.args.join(' ')}`
      );

      const proc = spawn(request.cliPath, request.args, {
        cwd: workspaceRoot,
        env: process.env,
      });

      console.log(
        `[ProcessCoordinator ${this.instanceId}] [#${request.id}] Process spawned with PID=${proc.pid}`
      );

      // Track this process
      this.currentProcess = proc;
      request.state = 'running';

      let stdout = '';
      let stderr = '';

      proc.stdout.on('data', (data: Buffer) => {
        stdout += data.toString();
      });

      proc.stderr.on('data', (data: Buffer) => {
        stderr += data.toString();
      });

      proc.on('close', (code: number | null) => {
        console.log(
          `[ProcessCoordinator ${this.instanceId}] [#${request.id}] Process PID=${proc.pid} closed with code=${code}`
        );

        // Clear the process reference
        if (this.currentProcess?.pid === proc.pid) {
          this.currentProcess = null;
        }

        if (code === 0) {
          resolve(stdout);
        } else if (code === null) {
          reject(new Error('Command was cancelled'));
        } else {
          const errorMessage = stderr.trim() || `CLI exited with code ${code}`;
          reject(new Error(errorMessage));
        }
      });

      proc.on('error', (err: Error) => {
        console.error(
          `[ProcessCoordinator ${this.instanceId}] [#${request.id}] Process error: ${err.message}`
        );

        // Clear the process reference
        if (this.currentProcess?.pid === proc.pid) {
          this.currentProcess = null;
        }

        reject(err);
      });
    });
  }

  private async waitForLockFile(requestId: number): Promise<void> {
    const lockFile = this.getLockFilePath();
    let waited = 0;

    while (fs.existsSync(lockFile) && waited < 3000) {
      if (waited === 0) {
        try {
          const lockContent = fs.readFileSync(lockFile, 'utf-8');
          console.log(
            `[ProcessCoordinator ${this.instanceId}] [#${requestId}] Lock file EXISTS with PID: ${lockContent}`
          );
        } catch (_err) {
          console.log(
            `[ProcessCoordinator ${this.instanceId}] [#${requestId}] Lock file EXISTS but can't read`
          );
        }
      }
      await new Promise((resolve) => setTimeout(resolve, 100));
      waited += 100;

      if (waited % 500 === 0) {
        console.log(
          `[ProcessCoordinator ${this.instanceId}] [#${requestId}] Still waiting for lock file after ${waited}ms`
        );
      }
    }

    if (fs.existsSync(lockFile)) {
      // Lock file still exists after timeout - this is a real lock conflict
      throw new Error(
        'Another renamify process is already running (lock file timeout)'
      );
    }
  }

  private async killCurrentProcess(requestId: number): Promise<void> {
    if (!this.currentProcess) {
      console.log(
        `[ProcessCoordinator ${this.instanceId}] [#${requestId}] No current process to kill`
      );
      return;
    }

    const proc = this.currentProcess;
    console.log(
      `[ProcessCoordinator ${this.instanceId}] [#${requestId}] Killing process PID=${proc.pid}`
    );

    // Send SIGTERM first
    proc.kill('SIGTERM');

    // Wait for graceful exit - give SIGTERM enough time to clean up lock files
    let waited = 0;
    while (waited < 2000 && proc.exitCode === null) {
      await new Promise((resolve) => setTimeout(resolve, 50));
      waited += 50;
    }

    if (proc.exitCode === null) {
      console.log(
        `[ProcessCoordinator ${this.instanceId}] [#${requestId}] Process didn't exit gracefully, sending SIGKILL`
      );
      proc.kill('SIGKILL');

      // Wait for SIGKILL to take effect
      while (waited < 3000 && proc.exitCode === null) {
        await new Promise((resolve) => setTimeout(resolve, 50));
        waited += 50;
      }
    }

    console.log(
      `[ProcessCoordinator ${this.instanceId}] [#${requestId}] Process terminated (exit code: ${proc.exitCode}, waited: ${waited}ms)`
    );
    this.currentProcess = null;

    // Wait for lock file cleanup
    const lockFile = this.getLockFilePath();
    waited = 0;
    while (waited < 1000 && fs.existsSync(lockFile)) {
      await new Promise((resolve) => setTimeout(resolve, 50));
      waited += 50;
    }

    if (fs.existsSync(lockFile)) {
      console.warn(
        `[ProcessCoordinator ${this.instanceId}] [#${requestId}] Lock file still exists after killing process - this should be rare since CLI handles SIGTERM properly`
      );
      // Lock file should be cleaned up by the CLI's Drop destructors when it receives SIGTERM
      // If it still exists, the process may have been SIGKILL'd or crashed
    }
  }

  async killCurrentCommand(): Promise<void> {
    console.log(
      `[${new Date().toISOString()}] [ProcessCoordinator ${this.instanceId}] killCurrentCommand: CALLED`
    );

    // Prevent concurrent kill operations
    if (this.isKilling) {
      console.log(
        `[${new Date().toISOString()}] [ProcessCoordinator ${this.instanceId}] killCurrentCommand: ALREADY KILLING - IGNORING`
      );
      return;
    }

    this.isKilling = true;

    try {
      // Cancel any pending request immediately
      if (this.pendingRequest) {
        console.log(
          `[${new Date().toISOString()}] [ProcessCoordinator ${this.instanceId}] killCurrentCommand: CANCELLING PENDING REQUEST #${this.pendingRequest.id}`
        );
        this.pendingRequest.state = 'cancelled';
        this.pendingRequest.resolve(null);
        this.pendingRequest = null;
      }

      // Kill active process if any
      if (this.currentProcess && this.activeRequest) {
        console.log(
          `[${new Date().toISOString()}] [ProcessCoordinator ${this.instanceId}] killCurrentCommand: KILLING ACTIVE REQUEST #${this.activeRequest.id}`
        );
        await this.killCurrentProcess(this.activeRequest.id);
      }
    } finally {
      this.isKilling = false;
    }
  }
}
