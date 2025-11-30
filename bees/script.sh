echo "===== FILE: Cargo.toml ====="
cat Cargo.toml
echo
find src -type f -name "*.rs" | sort | while read -r file; do
  echo "===== FILE: $file ====="
  cat "$file"
  echo -e "\n"
done
