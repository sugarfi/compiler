module.exports = {
    purge: ["**/*.html"],
    outDir: "dist",
    postcss: [ require("autoprefixer") ],
}
