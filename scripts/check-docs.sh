#!/usr/bin/env sh
set -eu

status=0

for document in README ARCHITECTURE DEVELOPMENT GAMES GIT_WORKFLOW RELEASING TRANSLATIONS; do
  source_file="docs/${document}.md"
  if [ ! -f "$source_file" ]; then
    echo "Missing canonical document: $source_file" >&2
    status=1
  fi

  for locale in ko zh-CN; do
    translated_file="docs/${locale}/${document}.md"
    if [ ! -f "$translated_file" ]; then
      echo "Missing ${locale} translation: $translated_file" >&2
      status=1
    fi
  done
done

for document in CONTRIBUTING CODE_OF_CONDUCT GOVERNANCE SECURITY SUPPORT; do
  source_file="${document}.md"
  if [ ! -f "$source_file" ]; then
    echo "Missing community document: $source_file" >&2
    status=1
  fi

  for locale in ko zh-CN; do
    translated_file="docs/${locale}/${document}.md"
    if [ ! -f "$translated_file" ]; then
      echo "Missing ${locale} translation: $translated_file" >&2
      status=1
    fi
  done
done

for locale in ko zh-CN; do
  if [ ! -f "README.${locale}.md" ]; then
    echo "Missing localized project README: README.${locale}.md" >&2
    status=1
  fi
done

exit "$status"
