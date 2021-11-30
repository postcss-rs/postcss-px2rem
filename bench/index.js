const fs = require("fs");
const postcss = require("postcss");
const pxtorem = require("postcss-pxtorem");
const css = fs.readFileSync("../assets/bootstrap.css", "utf8");
const options = {
  replace: false
};

console.time('postcss')
const processedCss = postcss(pxtorem(options)).process(css).css;
console.timeEnd('postcss')
