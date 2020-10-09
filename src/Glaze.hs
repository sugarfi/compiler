module Glaze where

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

compileFile :: String -> IO ()
compileFile file = do
    nodes <- parseFile file
    writeFile "example.css" $ compile $ generate nodes
