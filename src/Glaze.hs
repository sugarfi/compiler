module Glaze where

import Glaze.AST
import Glaze.Parser

import Text.ParserCombinators.Parsec

parseInput :: String -> Expr
parseInput str =
    case parse parseExpr "" str of
        Left e  -> error $ show e
        Right r -> r
