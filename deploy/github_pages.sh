rm -rf docs/*
dx bundle --out-dir docs
mv docs/public/* docs
cp docs/index.html docs/404.html