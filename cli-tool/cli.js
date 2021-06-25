#!/usr/bin/env node
require('dotenv').config()
/* eslint-disable */
require('yargs/yargs')(process.argv.slice(2))
  .commandDir('cmds')
  .demandCommand()
  .help()
  .argv
