const path = require("path")
const native = require("../native")

module.exports = function(input, output) {
	if (!output) output = "."

	input = path.resolve(process.cwd(), input)
	output = path.resolve(process.cwd(), output)

	console.log(native.compile(input, output))
}
