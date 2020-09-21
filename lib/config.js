const fs = require("fs")
const path = require("path")
const os = require("os")
const Joi = require("joi")

module.exports = function (input) {
	// Find config in current directory or parent directories

	let configLoc = path.resolve(process.cwd(), path.dirname(input))
	let found = false

	while (true) {
		if (fs.existsSync(path.join(configLoc, "glaze.config.js"))) {
			configLoc = path.join(configLoc, "glaze.config.js")
			found = true
			break
		}
		if (configLoc === os.homedir()) break
		configLoc = path.resolve(configLoc, "..")
	}

	// This is very slow (>800ms), perhaps there is a better alternative
	// for loading JS files than require?
	const config = found ? require(configLoc) : {}

	// Verify schema

	const schema = Joi.object({
	    purge: Joi.array().items(Joi.string()),
	    outDir: Joi.string(),
	    postcss: Joi.array().items(Joi.function()),
	})

	const { error } = schema.validate(config)
	if (error) {
	    console.error(error)
	    process.exit()
	}

	// Return with defaults

	return {
	    purge: config.purge || [],
	    outDir: config.outDir || ".",
	    postcss: config.postcss || [],
	}
}
