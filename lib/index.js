#!/usr/bin/env node
/*
 * Copyright (C) 2020 GiraffeKey
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the GNU General Public License as published by
 * the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the GNU General Public License
 * along with this program.  If not, see <https://www.gnu.org/licenses/>.
 */


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
