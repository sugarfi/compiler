#!/usr/bin/env node

const { Command } = require("commander")
const compile = require("./compile")
const init = require("./init")
const pjson = require("../package.json")

const program = new Command()

program
	.version(pjson.version, "-v, --version")
	.arguments("<input_file> [output_dir]")
	.option("-p, --production", "Enables production mode")
	.action(compile)

program
	.command("init [dir]")
	.action(init)
	
program.parse(process.argv)
