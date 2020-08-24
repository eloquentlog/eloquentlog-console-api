INSTALL
=======

Build
~~~~~

.. code:: zsh

   # e.g. server
   $ IMAGE_NAME=eloquentlog-console-api-server

   % docker build --file Dockerfile \
     --build-arg BINARY=server \
     --tag ${IMAGE_NAME}:latest .

   # e.g. Cloud Registry on Google Cloud Platform
   # - https://cloud.google.com/container-registry/docs/advanced-authentication
   # - https://github.com/GoogleCloudPlatform/docker-credential-gcr
   % gcloud auth configure-docker

   # or use docker-credential-gcr
   % VERSION=...
   % OS=linux
   % ARCH=amd64
   % curl -fsSL "https://.../v${VERSION}/..._${OS}_${ARCH}-${VERSION}.tar.gz" \
     | tar xz --to-stdout ./docker-credential-gcr \
     > .tool/docker-credential-gcr && \
     chmod +x .tool/docker-credential-gcr
   % .tool/docker-credential-gcr configure-docker

   # push
   % PROJECT_ID=...
   % HOST=eu.gcr.io
   % docker tag ${IMAGE_NAME}:latest ${HOST}/${PROJECT_ID}/${IMAGE_NAME}:latest
   % docker push ${HOST_NAME}/${PROJECT_ID}/${IMAGE_NAME}:latest


GCP
~~~

This is experimental. The following middlewares are required.

* ``Cloud SQL`` for PostgreSQL
* ``Memorystore`` for Redis (with VPC connector)
* ``Cloud Storage`` (for logging)

The ``server`` and ``worker`` both applications work with same environment
variablse. The build will be done via ``Cloud Build``.

Cloud Run
^^^^^^^^^

target: ``server``

0. Setup gcloud on local
........................

.. code:: zsh

   % .tool/setup-cloud-sdk
   % source .tool/load-gcloud

1. Prepare env vars for applications
....................................

.. code:: zsh

   % cp .env.deploy.sample .env.deploy
   % $EDITOR .env.deploy

There is some note for the postgres connection via unix socket.
``/.s.PGSQL.5432`` will be appended automatically, and slash and colon must be
escaped in DATABASE_URL. So, it should look like:
``DATABASE_URL="postgresql://user:password@%2Fpath%2Fto%2Fdir%3Afoo%3Abar``

2. Prepare env vars for deploy
..............................

Especially ``GCP_CLOUD_BUILD_SUBSTR_ENV_VARS`` must be a path to the file
``.env.deploy`` which has been created in above step.

.. code:: zsh

   % cp .env.ci.sample .env.ci
   % $EDITOR .env.ci

   # each line has export (for make)
   % source .env.ci

3. Run make deploy
..................

Currently, it may take more than 30 minutes...

.. code:: zsh

   % make deploy:server


Compute Engine
^^^^^^^^^^^^^^

target: ``server``, ``worker``

0. Create instance with a container
...................................

.. code:: zsh

   % gcloud compute instances create-with-container <NAME> \
     --container-image <IMAGE> \
     --zone <ZONE> \
     --container-restart-policy always \
     --container-env-file .env.worker

   % gcloud compute instances set-scopes <NAME> \
      --scopes=sql-admin,default

   # remove unnecessary external IP address
   % gcloud compute addresses delete <ADDRESS>


