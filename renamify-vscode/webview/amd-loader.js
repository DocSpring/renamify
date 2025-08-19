// Minimal AMD loader for VS Code webview
(() => {
  const modules = {};
  const waiting = {};

  window.define = (nameParam, depsParam, factoryParam) => {
    let name = nameParam;
    let deps = depsParam;
    let factory = factoryParam;

    if (typeof name !== 'string') {
      // Anonymous module
      factory = deps;
      deps = name;
      name = null;
    }

    if (!Array.isArray(deps)) {
      factory = deps;
      deps = ['require', 'exports'];
    }

    const module = { exports: {} };
    const depModules = deps.map((dep) => {
      if (dep === 'require') {
        return window.require;
      }
      if (dep === 'exports') {
        return module.exports;
      }
      return modules[dep];
    });

    factory(...depModules);

    if (name) {
      modules[name] = module.exports;

      // Check if anything was waiting for this module
      if (waiting[name]) {
        for (const callback of waiting[name]) {
          callback(module.exports);
        }
        delete waiting[name];
      }
    }
  };

  window.require = (deps, callback) => {
    if (typeof deps === 'string') {
      return modules[deps];
    }

    if (callback) {
      const depModules = deps.map((dep) => modules[dep]);
      callback(...depModules);
    }
  };

  // For compatibility
  window.define.amd = {};
})();
