# Possible requirements

apt-get install python3-venv

# Deploying to Read the Docs

RtD builds and deploys new versions of the docs whenever you push a new commit
to GitHub (though you can tag it with a specific release!)

# Running locally

./run_local.sh && firefox /tmp/grapl_docs/index.html

# PSA:

This section cannot be migrated to Pants yet, because of how ReadTheDocs
currently builds it. We could explore switching to a static build.
