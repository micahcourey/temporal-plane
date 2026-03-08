# API Specification

| Field | Value |
|-------|-------|
| **Service Name** | [service-name-api] |
| **Base URL** | `/api/v1/resource` |
| **Version** | [1.0.0] |
| **Author** | [Name] |
| **Last Updated** | [YYYY-MM-DD] |

---

## Overview

[Brief description of the API — what domain it covers, who the consumers are.]

## Authentication

All endpoints require a valid JWT token in the `Authorization` header:

```
Authorization: Bearer <token>
```

Tokens are obtained via Okta SSO and validated by `authMiddleware.verifyTokenRetrieveUser`.

## Common Headers

| Header | Required | Description |
|--------|----------|-------------|
| `Authorization` | Yes | Bearer token |
| `Content-Type` | Yes (POST/PUT) | `application/json` |
| `X-Request-Id` | No | Correlation ID for tracing |

## Error Response Format

All errors follow a standard structure:

```json
{
  "error": "ERROR_CODE",
  "message": "Human-readable description",
  "statusCode": 400
}
```

### Standard Error Codes

| HTTP Status | Error Code | Description |
|-------------|------------|-------------|
| 400 | `BAD_REQUEST` | Invalid input or missing required fields |
| 401 | `UNAUTHORIZED` | Missing or invalid token |
| 403 | `FORBIDDEN` | Insufficient privileges |
| 404 | `NOT_FOUND` | Resource does not exist |
| 409 | `CONFLICT` | Duplicate or state conflict |
| 500 | `INTERNAL_ERROR` | Unexpected server error |

---

## Endpoints

### List Resources

```
GET /api/v1/resource
```

**Privilege**: `VIEW_RESOURCE`

**Query Parameters**:

| Parameter | Type | Required | Default | Description |
|-----------|------|----------|---------|-------------|
| `page` | integer | No | 1 | Page number |
| `pageSize` | integer | No | 25 | Results per page |
| `sortBy` | string | No | `created_dt` | Sort field |
| `sortOrder` | string | No | `desc` | `asc` or `desc` |
| `search` | string | No | | Full-text search term |

**Response** `200 OK`:

```json
{
  "data": [
    {
      "id": 1,
      "name": "Example",
      "status": "ACTIVE",
      "createdDt": "2025-01-15T10:30:00Z",
      "createdByUserId": 42
    }
  ],
  "pagination": {
    "page": 1,
    "pageSize": 25,
    "totalRecords": 100,
    "totalPages": 4
  }
}
```

---

### Get Resource by ID

```
GET /api/v1/resource/:id
```

**Privilege**: `VIEW_RESOURCE`

**Path Parameters**:

| Parameter | Type | Description |
|-----------|------|-------------|
| `id` | integer | Resource ID |

**Response** `200 OK`:

```json
{
  "id": 1,
  "name": "Example",
  "description": "Detailed description",
  "status": "ACTIVE",
  "agreementId": 5,
  "createdDt": "2025-01-15T10:30:00Z",
  "createdByUserId": 42,
  "modifiedDt": null,
  "modifiedByUserId": null
}
```

**Errors**: `404 NOT_FOUND` if resource does not exist or belongs to a different ACO.

---

### Create Resource

```
POST /api/v1/resource
```

**Privilege**: `CREATE_RESOURCE`

**Request Body**:

```json
{
  "name": "New Resource",
  "description": "Description text",
  "agreementId": 5
}
```

| Field | Type | Required | Validation |
|-------|------|----------|------------|
| `name` | string | Yes | 1-255 chars |
| `description` | string | No | Max 2000 chars |
| `agreementId` | integer | Yes | Must match user's ACO |

**Response** `201 Created`:

```json
{
  "id": 2,
  "name": "New Resource",
  "description": "Description text",
  "status": "ACTIVE",
  "agreementId": 5,
  "createdDt": "2025-01-16T08:00:00Z",
  "createdByUserId": 42
}
```

**Errors**: `400 BAD_REQUEST` for validation failures.

---

### Update Resource

```
PUT /api/v1/resource/:id
```

**Privilege**: `EDIT_RESOURCE`

**Request Body**:

```json
{
  "name": "Updated Name",
  "description": "Updated description"
}
```

**Response** `200 OK`: Returns the updated resource.

**Errors**: `404 NOT_FOUND`, `400 BAD_REQUEST`.

---

### Delete Resource

```
DELETE /api/v1/resource/:id
```

**Privilege**: `DELETE_RESOURCE`

**Response** `204 No Content`

**Errors**: `404 NOT_FOUND`, `409 CONFLICT` if resource is referenced by other entities.

---

## Data Isolation

All endpoints automatically filter by `agreement_id` based on the authenticated user's ACO assignment. Users cannot access resources belonging to other ACOs.

## Rate Limiting

| Tier | Limit |
|------|-------|
| Standard | 100 requests/minute |
| Batch | 20 requests/minute |

## Changelog

| Version | Date | Changes |
|---------|------|---------|
| 1.0.0 | [Date] | Initial release |
