const fs = require("fs")
const path = require("path")
const postcss = require("postcss")
const { PurgeCSS } = require("purgecss")
const csso = require("csso")
const terser = require("terser")
const getConfig = require("./config")
const native = require("../native")

module.exports = async function (input, output, cmdObj) {
    const config = getConfig(input)

    if (!output) output = config.outDir
    if (path.extname(input) !== ".glz") {
    	console.error("Glaze files must end with .glz extension")
    	process.exit()
    }

    input = path.resolve(process.cwd(), input)
    output = path.resolve(process.cwd(), output)

    let { css, js } = native.compile(input)

    if (config.postcss.length) {
        css = (await postcss(config.postcss)
            .process(css, { from: undefined })).css
    }

    if (cmdObj.production) {
    	if (config.purge.length) {
    		const purgecss = new PurgeCSS()

    		css = (await purgecss.purge({
    			content: config.purge,
    			css: [{ raw: css }],
    		}))[0].css
    	}

    	css = csso.minify(css).css
    	// js = (await terser.minify(js)).code
    }

    fs.mkdirSync(path.join(output), { recursive: true })
    const fd = fs.openSync(path.join(output, path.basename(input).split(".")[0]) + ".css", "w")
    fs.writeSync(fd, css, 0, "utf8")
}
