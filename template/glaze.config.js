module.exports = {
    purge: ["**/*.html"],
    outDir: "dist",
    postcss: {
        plugins: [
            require("autoprefixer"),
        ],
    },
}
