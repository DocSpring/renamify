module.exports = {
  inputDir: "icons-src",
  outputDir: "dist",
  name: "renamify-glyphs",
  fontTypes: ["woff2"],
  assetTypes: [], // gets you a codepoint map
  normalize: true,
  descent: 128, // tweak until it looks vertically centered
  // Pin codepoints so they never change
  codepoints: {
    renamify: 0xf113, // private-use area
  },
};
