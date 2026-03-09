# aamva-web

A very lenient parser for barcodes as specified by AAMVA for North American
identification cards.

## Example

```typescript
import { decodeBarcode } from "@syfaro/aamva-web";

const data = ""; // Scan, etc. your barcode data

const id = decodeBarcode(data);
console.log(id.date_of_birth);
```
