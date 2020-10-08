# Reactive

## Subscriptions

You can subscribe to another property like so:

```glaze
img
	width: 10%
	height: &.width
```

Now the image will always be square.

Note: width can even be set to auto

## Events

```glaze
.item
	@data
		highlighted: false

	if &.data.highlighted
		border: 2px solid green

	display: flex

	p
		flex-grow: 1

	button
		if ~/.data.highlighted
			color: green
		else
			color: blue

		@click
			~/
				@data
					highlighted: not &.data.highlighted
```
