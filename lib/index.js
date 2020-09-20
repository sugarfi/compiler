#!/usr/bin/env node

const { Command } = require("commander")
const { version } = require("../package.json")

const program = new Command()
const config = require("./config")

program
    .version(version, "-v, --version")
    .arguments("<input_file> [output_dir]")
    .option("-p, --production", "Enables production mode")
    .action(require("./compile")(config))

program
    .command("init [dir]")
    .action(require("./init")(config))

program.parse(process.argv)
