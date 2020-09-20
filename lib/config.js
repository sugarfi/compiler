const fs = require("fs")
const path = require("path")
const Joi = require("joi")

const configLoc = path.resolve(process.cwd(), "glaze.config.js")
const config = fs.existsSync(configLoc) ? require(configLoc) : {}

const schema = Joi.object({
    purge: Joi.array().items(Joi.string()),
    outDir: Joi.string(),
    postcss: Joi.object(),
})

const { error } = schema.validate(config)
if (error) {
    console.error(error)
    process.exit()
}

module.exports = {
    purge: config.purge || [],
    outDir: config.outDir || ".",
    postcss: config.postcss || {},
}
