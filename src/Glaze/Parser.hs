module Glaze.Parser where

import Glaze.AST

import Control.Applicative (liftA2)
import Control.Monad

import Text.Parsec.Number
import Text.ParserCombinators.Parsec

lexeme :: Parser a -> Parser a
lexeme p = p <* spaces

parseExpr :: Parser Expr
parseExpr =  parseBool
         <|> parseString
         <|> parseHex
         <|> parseDimension
         <|> parseNumber
         <|> parseVariable
         <|> parseCall
         <|> parseSymbol
         <|> parseTuple
         <|> parseList
         <|> parseRecord

parseNumber :: Parser Expr
parseNumber = lexeme $ ExprNumber <$> rawNumber

rawNumber :: Parser Float
rawNumber = ap sign $ floating3 True

parseString :: Parser Expr
parseString =
    lexeme $ ExprString <$> string
    where
        string =
            char '"'
            *>
            (many $ noneOf ['"'])
            <*
            char '"'

parseBool :: Parser Expr
parseBool = 
    lexeme $ ExprBool <$> (true <|> false) <* notFollowedBy alphaNum
    where
        true  = (string "true")  *> (pure True)
        false = (string "false") *> (pure False)

parseSymbol :: Parser Expr
parseSymbol = lexeme $ ExprSymbol <$> rawSymbol

rawSymbol :: Parser String
rawSymbol =
    liftA2 (:)
    (letter <|> oneOf "-_\\")
    (many (alphaNum <|> oneOf "-_\\"))

parseHex :: Parser Expr
parseHex =
    lexeme $ ExprHex <$> hex
    where
        hex =
            char '#'
            *>
            many1 hexDigit

parseDimension :: Parser Expr
parseDimension =
    try $ lexeme $ ExprDimension <$> dimension
    where
        dimension = do
            v <- rawNumber
            u <- rawSymbol
            return (v, u)

parseTuple :: Parser Expr
parseTuple = lexeme $ ExprTuple <$> rawTuple

rawTuple :: Parser [Expr]
rawTuple =
    (lexeme $ char '(')
    *>
    (parseExpr `sepBy` (lexeme $ char ','))
    <*
    (lexeme $ char ')')

parseList :: Parser Expr
parseList =
    lexeme $ ExprList <$> list
    where
        list =
            (lexeme $ char '[')
            *>
            (parseExpr `sepBy` (lexeme $ char ','))
            <*
            (lexeme $ char ']')

parseRecord :: Parser Expr
parseRecord =
    lexeme $ ExprRecord <$> record
    where
        record =
            (lexeme $ char '{')
            *>
            (entry `sepBy` (lexeme $ char ','))
            <*
            (lexeme $ char '}')
            where
                entry = do
                    key <- lexeme $ rawSymbol
                    (lexeme $ char ':')
                    value <- parseExpr
                    return (key, value)

parseVariable :: Parser Expr
parseVariable =
    lexeme $ ExprVariable <$> variable
    where
        variable =
            char '$'
            *>
            rawSymbol

parseCall :: Parser Expr
parseCall =
    try $ lexeme $ ExprFunction <$> function
    where
        function = do
            name <- rawSymbol
            args <- rawTuple
            return (name, args)
