# PULQVA UX Contract v1

The product is judged from the user's first minute.

## Primary path
1. User obtains one supported release package.
2. User opens/unpacks it.
3. User launches PULQVA without installing Python, Node, Rust, Tor, yt-dlp, FFmpeg, or an AI model.
4. PULQVA establishes the required privacy transport.
5. User types a natural-language request.
6. PULQVA interprets intent without requiring an account or API key.
7. PULQVA searches and presents useful candidates.
8. User chooses a candidate and clicks Download, or chooses Autopilot.
9. PULQVA saves the resulting file in a predictable user location.
10. PULQVA clearly reports success or a specific failure.

## No-surprise rules
- No hidden clearnet fallback.
- No surprise login wall in the default path.
- No API-key setup in the default path.
- No terminal required for ordinary use.
- No installer chain for ordinary use.
- No prompt that asks the user to understand Tor, yt-dlp, FFmpeg, or model plumbing.
- Advanced controls may exist, but cannot obstruct the primary path.

## Success metric
A technically non-expert user should be able to understand the main interaction without reading documentation.
