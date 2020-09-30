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

const fs = require("fs")
const path = require("path")
const { spawn } = require("child_process")
const { ncp } = require("ncp")

module.exports = function (dir) {
    // Defaults

    if (!dir) dir = "."

    let command
    if (fs.existsSync(path.resolve(process.cwd(), "yarn.lock"))) {
    	command = ["yarn", ["add", "-D", "autoprefixer"]]
    } else {
    	command = ["npm", ["i", "-D", "autoprefixer"]]
    }
	
    const child = spawn(...command)
    child.stdout.pipe(process.stdin)
    child.stderr.pipe(process.stdin)

    const from = path.resolve(__dirname, "../template")
    const to = path.resolve(process.cwd(), dir)

    ncp(from, to)
}
