Eloquentlog Console API
=======================

.. image:: https://gitlab.com/eloquentlog/eloquentlog-console-api/badges/trunk/pipeline.svg
   :target: https://gitlab.com/eloquentlog/eloquentlog-console-api/commits/trunk

.. image:: https://gitlab.com/eloquentlog/eloquentlog-console-api/badges/trunk/coverage.svg
   :target: https://gitlab.com/eloquentlog/eloquentlog-console-api/commits/trunk

.. code:: text

   Eloquentlog

   ╔═╗┌─┐┌┐┌┌─┐┌─┐┬  ┌─┐  ╔═╗╔═╗╦
   ║  │ ││││└─┐│ ││  ├┤   ╠═╣╠═╝║
   ╚═╝└─┘┘└┘└─┘└─┘┴─┘└─┘  ╩ ╩╩  ╩

The console backend API server of Eloquentlog_.


Repositories
------------

| The main repository_ is hosted on GitLab.com.
| # A mirror_ is available also on GitHub.


Requirements
------------

Build
~~~~~

* Rust Nightly `>= 2021-05-11`

Runtime
~~~~~~~

* PostgreSQL (uuid)
* Redis


Setup
-----

.. code:: zsh

   : set env variables for your environment
   % cp .env{.sample,}

   : before this, give SUPERUSER to migration user for `CREATE EXTENSION`
   % make schema:migration:commit


Build
-----

.. code:: zsh

   : debug
   % make build

   : see help for building
   % make help | grep 'build '
   ...


Docker
~~~~~~

See `docker-compose.yml`. Alternatively, you can build and run them manually.


.. code:: zsh

   : prepare .env file
   % cp .env{.sample,}

.. code:: zsh

   : server
   % docker build --file Dockerfile \
     --build-arg BINARY=server \
     --tag eloquentlog/eloquentlog-console-api-server:latest .
   % docker run --env_file ./.env \
     -it eloquentlog/eloquentlog-console-api-server:latest

   : worker
   % docker build --file Dockerfile \
     --build-arg BINARY=worker \
     --tag eloquentlog/eloquentlog-console-api-worker:latest .
   % docker run --env_file ./.env \
     -it eloquentlog/eloquentlog-console-api-worker:latest


As a common issue, ``--env_file`` doesn't handle double-quoted string like
``FOO="bar"`` because it's not evaluated via shell.


.. code:: zsh

   : this might be something help if you want to connect to host from guest
   % alias host="ip route show 0.0.0.0/0 | grep -Eo 'via \S+' | awk '{print \$2}'"
   % docker run --add-host=postgres:$(host) --add-host=redis:$(host) \
     --env-file ./.env \
     --rm -it eloquentlog/eloquentlog-console-api-server


Development
-----------

Verify
~~~~~~

.. code:: zsh

   : see make help about details
   % make verify

Route
~~~~~

To check current routes, run `make route`.

.. code:: zsh

   : print all routes
   % make route
   ...

   : or build router separately
   % make build:router
   % ./target/debug/router
   ...

Run
~~~

Use cargo-watch_

.. code:: zsh

   % make watch:server
   % make watch:worker

   % curl \
     -H "Content-Type: application/json" \
     -H "Accept: application/json" \
     -d "{}" \
     -X POST \
     http://localhost:8000/_api/signin

Note
~~~~

Update Rust version
^^^^^^^^^^^^^^^^^^^

The current edition is ``2018``, and depends on nightly toolchain.

For build on CI, images by Portolan_ used. The tags format like below is
valid.

.. code:: txt

   1.48.0-nightly
   nightly-2021-05-11


To change the version, update following files:

* rust-toolchain
* .gitlab-ci.yml

New version release
^^^^^^^^^^^^^^^^^^^

TODO


Testing
-------

.. code:: zsh

   % ENV=test make schema:migration:commit

   % cargo test model::namespace::test -- --nocapture

   : run all
   % make test



Deployment
----------

See doc/INSTALL.rst


License
-------

.. code:: text

   ┏━╸╻  ┏━┓┏━┓╻ ╻┏━╸┏┓╻╺┳╸╻  ┏━┓┏━╸
   ┣╸ ┃  ┃ ┃┃┓┃┃ ┃┣╸ ┃┗┫ ┃ ┃  ┃ ┃┃╺┓
   ┗━╸┗━╸┗━┛┗┻┛┗━┛┗━╸╹ ╹ ╹ ┗━╸┗━┛┗━┛

   Console API
   Copyright (c) 2018-2021 Lupine Software LLC

``AGPL-3.0-or-later``.

.. code:: text

   This program is free software: you can redistribute it and/or modify
   it under the terms of the GNU Affero General Public License as published by
   the Free Software Foundation, either version 3 of the License, or
   (at your option) any later version.

   This program is distributed in the hope that it will be useful,
   but WITHOUT ANY WARRANTY; without even the implied warranty of
   MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
   GNU Affero General Public License for more details.

   You should have received a copy of the GNU Affero General Public License
   along with this program.  If not, see <https://www.gnu.org/licenses/>.


.. _Eloquentlog: https://eloquentlog.com/
.. _repository: https://gitlab.com/eloquentlog/eloquentlog-console-api
.. _mirror: https://github.com/eloquentlog/eloquentlog-console-api
.. _cargo-watch: https://github.com/passcod/cargo-watch
.. _Portolan: https://gitlab.com/grauwoelfchen/portolan
