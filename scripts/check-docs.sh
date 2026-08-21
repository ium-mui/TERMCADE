#!/usr/bin/env sh
set -eu

status=0

for document in README ARCHITECTURE DEVELOPMENT GAMES GIT_WORKFLOW RELEASING; do
  english_file="docs/${document}.md"
  if [ ! -f "$english_file" ]; then
    echo "Missing document: $english_file" >&2
    status=1
  fi

  for locale in ko zh-CN; do
    locale_file="docs/${locale}/${document}.md"
    if [ ! -f "$locale_file" ]; then
      echo "Missing ${locale} document: $locale_file" >&2
      status=1
    fi
  done
done

for document in CONTRIBUTING SECURITY; do
  english_file="${document}.md"
  if [ ! -f "$english_file" ]; then
    echo "Missing document: $english_file" >&2
    status=1
  fi

  for locale in ko zh-CN; do
    locale_file="docs/${locale}/${document}.md"
    if [ ! -f "$locale_file" ]; then
      echo "Missing ${locale} document: $locale_file" >&2
      status=1
    fi
  done
done

for locale in ko zh-CN; do
  if [ ! -f "README.${locale}.md" ]; then
    echo "Missing ${locale} project README: README.${locale}.md" >&2
    status=1
  fi
done

exit "$status"
