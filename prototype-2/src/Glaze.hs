module Main where

import Glaze.AST
import Glaze.Parser
import Glaze.Generator
import Glaze.Compiler

import Text.ParserCombinators.Parsec

parseFile :: String -> IO [Node]
parseFile file = do
    input <- readFile file
    case parse parseInput "" input of
        Left e  -> print e >> fail "parse error"
        Right r -> return r

main :: IO ()
main = do
    nodes <- parseFile "example.glz"
    print nodes
    let (css, js) = generate nodes
    print css
    print js
    let (csso, jso) = compile (css, js)
    writeFile "example.css" csso
    writeFile "example.js"  jso
