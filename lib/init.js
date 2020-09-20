const path = require("path")
const native = require("../native")

module.exports = function(dir) {
	if (!dir) dir = "."

	const from = path.resolve(__dirname, "../template")
	const to = path.resolve(process.cwd(), dir)

	native.copy(from, to)
}
