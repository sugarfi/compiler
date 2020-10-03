const fs = require("fs")
const path = require("path")

const dir = path.resolve("native/tests", process.argv[2])

fs.mkdirSync(dir, { recursive: true })

const files = ["style.glz", "style.css", "style.js"]

files.forEach(
	file => {
		let fd = fs.openSync(path.join(dir, file), "w")
		fs.writeSync(fd, "", 0, "utf8")
	}
)
