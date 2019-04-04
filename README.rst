Eloquentlog Backend API
=======================

.. image:: https://gitlab.com/eloquentlog/eloquentlog-backend-api/badges/master/pipeline.svg
   :target: https://gitlab.com/eloquentlog/eloquentlog-backend-api/commits/master

.. image:: https://gitlab.com/eloquentlog/eloquentlog-backend-api/badges/master/coverage.svg
   :target: https://gitlab.com/eloquentlog/eloquentlog-backend-api/commits/master

.. code:: text

   Eloquentlog

   ╔╗ ┌─┐┌─┐┬┌─┌─┐┌┐┌┌┬┐  ╔═╗╔═╗╦
   ╠╩╗├─┤│  ├┴┐├┤ │││ ││  ╠═╣╠═╝║
   ╚═╝┴ ┴└─┘┴ ┴└─┘┘└┘─┴┘  ╩ ╩╩  ╩

The backend API server of Eloquentlog_.


Repository
----------

https://gitlab.com/eloquentlog/eloquentlog-backend-api


Requirements
------------

* PostgreSQL
* Redis


Setup
-----

.. code:: zsh

    # set env variables for {production|testing|development}
    % cp .env.sample .env

    # give SUPERUSER to migration user for `CREATE EXTENSION`
    % make schema:migration:commit


Build
-----

.. code:: zsh

    # debug
    % make build


Development
-----------

Vet
~~~

.. code:: zsh

    # see make help about details
    % make vet

Run
~~~

Use cargo-watch_

.. code:: zsh

    % make watch:server
    % make watch:worker


Testing
-------

.. code:: zsh

    % ENV=test make schema:migration:commit

    % make test


License
-------

.. code:: text

   ┏━╸╻  ┏━┓┏━┓╻ ╻┏━╸┏┓╻╺┳╸╻  ┏━┓┏━╸
   ┣╸ ┃  ┃ ┃┃┓┃┃ ┃┣╸ ┃┗┫ ┃ ┃  ┃ ┃┃╺┓
   ┗━╸┗━╸┗━┛┗┻┛┗━┛┗━╸╹ ╹ ╹ ┗━╸┗━┛┗━┛

   Backend API
   Copyright (c) 2018-2019 Lupine Software LLC


`AGPL-3.0-or-later`


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
.. _cargo-watch: https://github.com/passcod/cargo-watch
