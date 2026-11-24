// A suite that never finishes: it holds a listening socket open and would
// only print its result once the socket closed.
import { createServer } from 'node:net';
import { done } from '../browser.mjs';
createServer().listen(0).on('close', () => done(0));
