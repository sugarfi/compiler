# Functions and Mixins

## Function definition

```glaze
add :: Number -> Number -> Number
add(a, b)
	a + b
```

## Calling

```glaze
p
	font-weight: add(200, 400)
```

## Mixin

Mixins are simply functions that return props.

```glaze
color-weight :: Hex -> Number -> Props
color-weight(c, w)
	color: c
	font-weight: w

p
	color-weight(#222, 400)
	color-weight: #222 400
```
