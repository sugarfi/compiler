const fs = require("fs")
const path = require("path")
const { spawn } = require("child_process")
const native = require("../native")

module.exports = function (config) {
    return function (dir) {
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

        native.copy(from, to)
    }
}
