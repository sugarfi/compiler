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
const postcss = require("postcss")
const { PurgeCSS } = require("purgecss")
const csso = require("csso")
const terser = require("terser")
const getConfig = require("./config")
const native = require("../native")

module.exports = async function (input, output, cmdObj) {
    // Load config

    const config = getConfig(input)

    // Defaults

    if (!output) output = config.outDir
    if (path.extname(input) !== ".glz") {
    	console.error("Glaze files must end with .glz extension")
    	process.exit()
    }

    input = path.resolve(process.cwd(), input)
    output = path.resolve(process.cwd(), output)

    // Generate CSS & JS

    let { css, js } = native.compile(input)

    // Purge and minify output

    if (cmdObj.production) {
    	if (config.purge.length) {
    		const purgecss = new PurgeCSS()

    		css = (await purgecss.purge({
    			content: config.purge,
    			css: [{ raw: css }],
    		}))[0].css
    	}

    	css = csso.minify(css).css
    	js = (await terser.minify(js)).code
    }

    // Write to file system

    fs.mkdirSync(path.join(output), { recursive: true })

    let fd = fs.openSync(path.join(output, path.basename(input).split(".")[0]) + ".css", "w")
    fs.writeSync(fd, css, 0, "utf8")
    
    fd = fs.openSync(path.join(output, path.basename(input).split(".")[0]) + ".js", "w")
    fs.writeSync(fd, js, 0, "utf8")
}
