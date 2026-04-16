#!/usr/bin/env node
import { program } from 'commander';
import React from 'react';
import { render } from 'ink';
import App from './app.js';

program
  .name('quack')
  .version('2.0.0')
  .option('--cmd <command>', 'Command to analyze')
  .action((options) => {
    render(React.createElement(App, { command: options.cmd }));
  });

program.parse();
