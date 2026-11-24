import { RESULT } from '../browser.mjs';
// A suite whose browser keeps the process alive after it prints its verdict (done() is the fix).
console.log(`${RESULT} pass`);
setInterval(() => {}, 1000);
