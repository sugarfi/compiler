module Glaze where

import Glaze.AST
import Glaze.Parser

import Text.ParserCombinators.Parsec

parseFile :: String -> IO [Node]
parseFile file = do
    input <- readFile file
    case parse parseInput "" input of
        Left e  -> print e >> fail "parse error"
        Right r -> return r

parseStr :: String -> Expr
parseStr str =
    case parse parseExpr "" str of
        Left e  -> error $ show e
        Right r -> r
