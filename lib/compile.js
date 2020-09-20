const path = require("path")
const native = require("../native")

module.exports = function (config) {
    return function (input, output) {
        if (!output) output = "."

        input = path.resolve(process.cwd(), input)
        output = path.resolve(process.cwd(), path.join(config.outDir, output))

        console.log(native.compile(input, output))
    }
}
