module Glaze.Parser where

import Glaze.AST

import Control.Applicative (liftA2)
import Control.Monad (ap)

import Data.List.Compat (singleton)

import Text.Parsec.Number
import Text.ParserCombinators.Parsec

(<||>) :: Parser a -> Parser a -> Parser a
(<||>) a b = try a <|> b

indent :: Int -> Parser String
indent n = count n tab <||> fmap concat (count n (string "    "))

ws :: Parser String
ws = many $ oneOf " \t"

nl :: Parser Char
nl = ws *> newline

parseInput :: Parser [Node]
parseInput = many parseRootNode <* eof

-- Nodes

parseRootNode :: Parser Node
parseRootNode =
    many nl *> ( parseDefinition 0
            <||> parseSelector   0
            <||> parseFunction
               )

parseNested :: Int -> Parser Node
parseNested n = parseDefinition n
           <||> parseProp       n

parseSelector :: Int -> Parser Node
parseSelector n =
    NodeSelector <$> selector
    where
        selector = do
            indent n
            sels <- sel `sepBy` (ws *> char ',' <* spaces)
            nl
            nodes <- many (parseNested (n + 1) <||> parseSelector (n + 1))
            return (sels, nodes)
            where
                sel =
                    fmap unwords
                    (fmap concat bit `sepBy` ws)
                    where
                        bit =
                            many1 ( rawSymbol
                               <||> fmap singleton (oneOf ".#:&/")
                                  )

parseFunction :: Parser Node
parseFunction =
    NodeFunction <$> function
    where
        function = do
            name <- rawSymbol
            char '('
            params <- (spaces *> rawSymbol) `sepBy` (spaces *> char ',')
            spaces *> char ')'
            ws *> string "::"
            types <- ((ws *> rawSymbol) `sepBy` (ws *> string "->"))
            nl
            nodes <- many $ parseNested 1
            return (name, params, nodes, types)

parseDefinition :: Int -> Parser Node
parseDefinition n =
    NodeDefinition <$> definition
    where
        definition = do
            indent n
            char '$'
            name <- rawSymbol
            spaces *> char '='
            value <- spaces *> parseExpr
            nl
            return (name, value)

parseProp :: Int -> Parser Node
parseProp n =
    NodeProp <$> (rawCall <||> prop)
    where
        prop = do
            indent n
            name <- rawSymbol
            spaces *> char ':'
            args <- spaces *> (parseExpr `sepBy` ws)
            nl
            return (name, args)

-- Expressions

parseExpr :: Parser Expr
parseExpr = parseBinaryOp 0
       <||> parseNonBinary

parseNonBinary :: Parser Expr
parseNonBinary = parseUnaryOp
            <||> parseBool
            <||> parseString
            <||> parseHex
            <||> parseDimension
            <||> parseNumber
            <||> parseVariable
            <||> parseCall
            <||> parseSymbol
            <||> parseTuple
            <||> parseList
            <||> parseRecord

parseNumber :: Parser Expr
parseNumber = ExprNumber <$> rawNumber

rawNumber :: Parser Float
rawNumber = ap sign $ floating3 True

parseString :: Parser Expr
parseString =
    ExprString <$> string
    where
        string =
            char '"'
            *>
            (many $ noneOf ['"'])
            <*
            char '"'

parseBool :: Parser Expr
parseBool = 
    ExprBool <$> (true <|> false) <* notFollowedBy alphaNum
    where
        true  = (string "true")  *> (pure True)
        false = (string "false") *> (pure False)

parseSymbol :: Parser Expr
parseSymbol = ExprSymbol <$> rawSymbol

rawSymbol :: Parser String
rawSymbol =
    liftA2 (:)
    (letter <|> oneOf "-_\\")
    (many (alphaNum <|> oneOf "-_\\"))

parseHex :: Parser Expr
parseHex =
    ExprHex <$> hex
    where
        hex =
            char '#' *> digits 6
            where
                digits 1 = count 1 hexDigit
                digits n = 
                    count n hexDigit
                    <||>
                    digits (n - 1)

parseDimension :: Parser Expr
parseDimension =
    ExprDimension <$> dimension
    where
        dimension = do
            v <- rawNumber
            u <- rawSymbol
            return (v, u)

parseTuple :: Parser Expr
parseTuple = ExprTuple <$> rawTuple

rawTuple :: Parser [Expr]
rawTuple =
    char '('
    *>
    ((spaces *> parseExpr) `sepBy` (spaces *> char ','))
    <*
    spaces <* char ')'

parseList :: Parser Expr
parseList =
    ExprList <$> list
    where
        list =
            char '['
            *>
            ((spaces *> parseExpr) `sepBy` (spaces *> char ','))
            <*
            spaces <* char ']'

parseRecord :: Parser Expr
parseRecord =
    ExprRecord <$> record
    where
        record =
            char '{'
            *>
            ((spaces *> entry) `sepBy` (spaces *> char ','))
            <*
            spaces <* char '}'
            where
                entry = do
                    key <- rawSymbol
                    spaces *> char ':'
                    value <- spaces *> parseExpr
                    return (key, value)

parseVariable :: Parser Expr
parseVariable =
    ExprVariable <$> variable
    where
        variable = char '$' *> rawSymbol

parseCall :: Parser Expr
parseCall =
    ExprFunction <$> rawCall

rawCall :: Parser (String, [Expr])
rawCall = do
    name <- rawSymbol
    args <- rawTuple
    return (name, args)

-- Operations

binaryOps = [ keyword "and" <||> keyword "or"
            , single "+" <||> single "-"
            , single "*" <||> single "/"
            ]
            where
                keyword :: String -> Parser String
                keyword a = many1 space *> string a <* many1 space

                single :: String -> Parser String
                single a = spaces *> string a <* spaces

parseBinaryOp :: Int -> Parser Expr
parseBinaryOp prec =
    if prec < length binaryOps - 1 then
        (ExprBinaryOp <$> binary) <||> parseBinaryOp (prec + 1)
    else
        ExprBinaryOp <$> binary
    where
        binary = do
            a  <- tryAll (prec + 1)
            op <- binaryOps !! prec
            b  <- tryAll prec
            return (op, a, b)
            where
                tryAll n =
                    if n < length binaryOps then
                        parseBinaryOp n <||> tryAll (n + 1)
                    else
                        parseNonBinary

parseUnaryOp :: Parser Expr
parseUnaryOp =
    ExprUnaryOp <$> unary
    where
        unary = do
            op <- symbol
            a <- parseExpr
            return (op, a)
            where
                symbol = string "not" <* many1 space
