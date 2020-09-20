# glaze

Glaze is an innovative and powerful UI programming language that compiles to CSS and JS. It was created to make UI programming more modular and stress-free by offering a [Stylus](https://github.com/stylus/stylus)-like syntax and package management system while removing the need to write JavaScript for UI-related code.

## Installation

`npm i -g glaze`

`yarn global add glaze`

## Basic Usage

Initiate a project:

`glaze init .`

Install a Glaze package:

`glaze add lost`

Compile output to the current directory:

`glaze ui/style.glz .`

There is much more to the Glaze CLI, check out the full API docs [here](https://glaze.dev/api/cli).

## Features

- Feature-rich CSS preprocessing
- Easy package management and modularity
- Event-driven UI updates
- Observables and subscriptions
- Lightweight preprocessing during runtime
- Linting and error reporting
- `!important` is illegal

## Example

The following will make sure all images within the .example class always have a height equal to their width. When a button within .example is clicked, the padding of all sibling images is doubled.

```glaze
.example
	img
		data:
			padding: 5

		padding: {data.padding}px
		width: {data.padding} * 20px
		height: {width}

	button
		click:
			images = $(& ~ img)
			for img in images
				img.data.padding *= 2
```

You can then register the above components like so:

```html
<div class="example">
	<img data-padding="5" />
	<button>Click Me!</button>
</div>
```

There is much more to Glaze, check out the full documentation [here](https://glaze.dev/docs).

## License

This repository is licensed under GPL 3.0. It should be noted that the GPL license only applies to the code in this repository. This means that you only need to GPL license your code if it is for some reason using source code from the CLI tool or compiler. Any Glaze code you write or compile does not have to be GPL licensed. Any modifications to the CLI tool or compiler, however, are expected to be contributed back to the open source community.
