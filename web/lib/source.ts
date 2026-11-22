import { docs, learn } from 'collections/server';
import { loader } from 'fumadocs-core/source';

export const source = loader({
  baseUrl: '/docs',
  source: docs.toFumadocsSource(),
});

export const learnSource = loader({
  baseUrl: '/learn',
  source: learn.toFumadocsSource(),
});
