# Liferay Mock Content Generator - Gemini Strategy

This file contains the mapping strategies and technical requirements for generating mock Web Content in Liferay using Gemini AI.

## Liferay Field Type Mapping (Headless Delivery API v1.0)

| Liferay Field Type | JSON Schema Type | Liferay Payload Format (`contentFieldValue`) | Notes |
| :--- | :--- | :--- | :--- |
| **Text** | `string` | `{"data": "string"}` | Short text fields. |
| **Rich Text** | `string` | `{"data": "<p>HTML string</p>"}` | Gemini should be prompted to provide HTML content. |
| **Numeric** | `number` | `{"data": 123}` | Integer or decimal numbers. |
| **Date** | `string` | `{"data": "YYYY-MM-DD"}` | Format must be ISO 8601 date. |
| **Boolean** | `boolean` | `{"data": true}` | True/False checkbox/toggle. |
| **Image** | `string` (URL) | `{"data": "image_url", "alt": "string"}` | Gemini can provide a placeholder URL (e.g., picsum.photos). |
| **Color** | `string` | `{"data": "#RRGGBB"}` | Hex code format. |
| **Select / Multiple** | `string` or `array` | `{"data": "val"}` or `{"data": ["v1", "v2"]}` | Values must match the structure's `options`. |

## JSON Schema Normalization Rules

1.  **Standard Fields**: Always include `title` (required) and `description` (optional) in the schema.
2.  **Field Names**: Use the `name` attribute from the Liferay Content Structure as the JSON key.
3.  **Required Fields**: By default, treat all fields in the structure as required to ensure high-quality mock data.

## Gemini Prompt Guidelines

- **Format**: Always request a JSON array of objects.
- **Strictness**: Provide the JSON Schema and instruct Gemini to follow it exactly.
- **Context**: For `Rich Text`, instruct Gemini to include relevant HTML tags (p, h2, ul, li) for a realistic look.
- **Images**: If an `image` field is present, ask Gemini to provide a descriptive prompt or a `picsum.photos` URL.

## Validation Strategy

1.  **Pre-Generation**: Fetch the structure definition and compile the JSON Schema.
2.  **Post-Generation**: Validate Gemini's output against the schema using `jsonschema`.
3.  **Pre-Submission**: Map valid objects into Liferay's `contentFields` array format.
