const path = require("path")
const Joi = require("joi")
const { cosmiconfigSync } = require("cosmiconfig")

module.exports = function (input) {
    // Load config

    const explorer = cosmiconfigSync("glaze")
    const { config } = explorer.search(path.resolve(process.cwd(), path.dirname(input)))
    if (config === null) config = {}

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
