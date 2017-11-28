==========
Changelog
==========

All notable changes to this project (especially breaking ones) will be
documented in this file.

This project adheres to `Semantic Versioning <https://semver.org/>`_.

Upcoming Changes
----------------

* Proper error handling using the ``Failure`` crate, introducing a
  ``TibrvError`` type to handle error conditions from Rendezvous.


`0.1.1`_ (2017-10-30)
---------------------

Initial public release.

* Added sync & async APIs for basic Rendezvous usage, along with convenience
  methods for encoding and decoding message fields.
* First usable release on `crates.io <https://crates.io/crates/tibrv>`_.


.. _`0.1.1`: https://github.com/bradfier/tibrv-rs/compare/2947f836...v0.1.1
