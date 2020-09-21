#!/usr/bin/env node

const { Command } = require("commander")
const { version } = require("../package.json")

const program = new Command()

program
    .version(version, "-v, --version")
    .arguments("<input_file> [output_dir]")
    .option("-p, --production", "Enables production mode")
    .action(require("./compile"))

program
    .command("init [dir]")
    .action(require("./init"))

program.parse(process.argv)
