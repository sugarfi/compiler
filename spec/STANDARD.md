# Standard

## Prelude

The prelude includes typed definitions for all CSS properties and functions.

## Higher-order functions

Example of map function

```glaze
$arr = 1, 2, 3, 4

p
	custom-prop: map(x -> $x + 1, $arr)
```

Outputs to

```css
p {
	custom-prop: 2, 3, 4, 5;
}
```
