# Co-Author Line Stripping Rules

## Matching

A line is considered a co-author trailer if its **trimmed** content starts with `co-authored-by:` (case-insensitive).

All of these match:

```
Co-authored-by: Name <email>
co-authored-by: Name <email>
CO-AUTHORED-BY: Name <email>
  Co-authored-by: Name <email>     (leading whitespace)
```

## Post-Removal Cleanup

After removing matched lines, trailing blank lines at the end of the message are collapsed. The result always ends with exactly one newline.

### Example

**Before:**
```
feat: add new feature

Some description.

Co-authored-by: Alice <alice@example.com>
Co-authored-by: Bob <bob@example.com>

```

**After:**
```
feat: add new feature

Some description.
```

## Edge Cases

### No co-author lines
The message is returned unchanged. The commit is not counted as "rewritten" in the summary.

### Co-author line in the middle of the body
Only the matching line is removed. Surrounding text is preserved:

**Before:**
```
title

paragraph one
Co-authored-by: Someone <s@e.com>
paragraph two
```

**After:**
```
title

paragraph one
paragraph two
```

### Message is only a co-author line
If removing all co-author lines would leave the message empty (no non-whitespace content), the tool aborts with an error. This prevents creating commits with empty messages.

### Git comment lines
Lines starting with `#` (git's comment syntax during rebase) are preserved — they are not co-author trailers. Git strips these itself after the editor exits.
