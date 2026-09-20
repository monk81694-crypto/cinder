// Keeps typecheck incremental: minimal `any` declarations for Node/Electron
// globals used by main.js, so tsc stays quiet without pulling in @types/node.
declare var require: any;
declare var module: any;
declare var __dirname: any;
declare var process: any;
