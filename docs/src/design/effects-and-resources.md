# Effects and resource management (proposal)

`yield` currently represents the value/control-flow boundary of a block. A
future design may generalize it into an effect handler for collections and
streams, allowing loops to yield multiple values.

Resource management also needs runtime support. A plain `close` after a block
cannot safely handle early returns or runtime errors; the implementation needs
unwind-safe cleanup, destructors, or an equivalent standard-library primitive.

These notes describe constraints for future design work. They are not APIs
available to Fobos programs today.
