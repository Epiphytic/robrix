# Matrix-Based Social Media Platform Architecture

## Overview

This document outlines how to build a semi-private social media platform by **reusing existing Matrix protocol structures**. No custom server modifications required—only client-side interpretation of standard Matrix primitives.

**Target Features:**
- Profile pages for users
- Posts (text, photos, videos, external links)
- Newsfeed across friend network
- Private and public events
- Broadcast lists for high-privacy scenarios

---

## Core Architecture Concept

```
┌─────────────────────────────────────────────────────────────┐
│                    User's Social Space                       │
│                   (@alice:example.org)                       │
├─────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │ Profile Room │  │ Public Feed  │  │ Friends-Only │       │
│  │ (state only) │  │    Room      │  │  Feed Room   │       │
│  └──────────────┘  └──────────────┘  └──────────────┘       │
│                                                              │
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐       │
│  │ Event: BBQ   │  │ Event: Bday  │  │  Friends     │       │
│  │  (public)    │  │  (private)   │  │   Space      │       │
│  └──────────────┘  └──────────────┘  └──────────────┘       │
└─────────────────────────────────────────────────────────────┘
```

---

## 1. Profile Pages

### Matrix Structures Used

| Feature | Matrix Primitive |
|---------|------------------|
| Display name | `m.presence` displayname / profile API |
| Online Status | `m.presence` (optional, configurable) |
| Avatar | Profile avatar_url (MXC URI) |
| Bio/About | State event in profile room |
| Links/Contact | State event in profile room |
| Cover photo | State event in profile room |

### Implementation

**User Profile Room** - Each user creates a dedicated "profile room" that acts as their profile page.

```json
{
  "preset": "public_chat",
  "name": "Alice's Profile",
  "room_alias_name": "profile_alice",
  "initial_state": [
    {
      "type": "m.room.topic",
      "content": { "topic": "Software developer | Coffee enthusiast" }
    },
    {
      "type": "org.social.profile",
      "state_key": "",
      "content": {
        "bio": "Building things with code",
        "location": "San Francisco",
        "website": "https://alice.dev",
        "cover_image": "mxc://example.org/cover123"
      }
    }
  ]
}
```

**Profile Discovery:**
- Room alias: `#profile_alice:example.org`
- Searchable via public room directory
- Profile room can be joined (follow) or just peeked (view)

### Presence Configuration

Reliability of `m.presence` varies by homeserver (Synapse supports it, some others don't).

| Requirement | Implementation |
|-------------|----------------|
| Client Handling | Treat Presence as an optional feature |
| User Control | Allow users to toggle presence on/off in settings |
| Unsupported Server | If homeserver returns 404 or presence is inactive, warn user that "Online" status will be disabled |
| Fallback | Use "Last Active" timestamps from Read Receipts (`m.read`) in feed rooms |

### Privacy Levels

| Level | Implementation |
|-------|----------------|
| Public | `join_rules: public`, `history_visibility: world_readable` |
| Semi-private | `join_rules: knock`, requires approval |
| Friends-only | `join_rules: restricted` to friends space |

---

## 2. Posts (Feed Content)

### Matrix Structures Used

| Content Type | Matrix Event |
|--------------|--------------|
| Text post | `m.room.message` with `msgtype: m.text` |
| Photo | `m.room.message` with `msgtype: m.image` |
| Video | `m.room.message` with `msgtype: m.video` |
| Link share | `m.room.message` with `msgtype: m.text` + URL preview |
| File share | `m.room.message` with `msgtype: m.file` |
| Mentions | `m.room.message` with `m.mentions` property |

### Post Structure (v1.17 Compliant)

> **Note:** As of v1.17, legacy body-based mentions are deprecated (MSC4210). Use `m.mentions` property.

**Text Post with Mention:**
```json
{
  "type": "m.room.message",
  "content": {
    "msgtype": "m.text",
    "body": "Hey @bob:example.org, check this out!",
    "format": "org.matrix.custom.html",
    "formatted_body": "<p>Hey <a href='https://matrix.to/#/@bob:example.org'>Bob</a>, check this out!</p>",
    "m.mentions": {
      "user_ids": ["@bob:example.org"],
      "room": false
    }
  }
}
```

**Photo Post:**
```json
{
  "type": "m.room.message",
  "content": {
    "msgtype": "m.image",
    "body": "Sunset at the beach",
    "url": "mxc://example.org/photo123",
    "info": {
      "mimetype": "image/jpeg",
      "w": 1920,
      "h": 1080,
      "size": 245000,
      "thumbnail_url": "mxc://example.org/thumb123"
    },
    "org.social.caption": "Beautiful sunset today! 🌅"
  }
}
```

**Video Post:**
```json
{
  "type": "m.room.message",
  "content": {
    "msgtype": "m.video",
    "body": "Concert highlight",
    "url": "mxc://example.org/video456",
    "info": {
      "mimetype": "video/mp4",
      "duration": 45000,
      "w": 1280,
      "h": 720,
      "thumbnail_url": "mxc://example.org/vthumb456"
    }
  }
}
```

**External Link Post:**
```json
{
  "type": "m.room.message",
  "content": {
    "msgtype": "m.text",
    "body": "Great article on distributed systems: https://example.com/article",
    "org.social.link_preview": {
      "url": "https://example.com/article",
      "title": "Understanding Distributed Systems",
      "description": "A deep dive into...",
      "image": "mxc://example.org/preview789"
    }
  }
}
```

### Engagement Features

| Feature | Matrix Primitive |
|---------|------------------|
| Likes/Reactions | `m.reaction` (annotation relationship) |
| Comments | `m.room.message` with `m.relates_to` (reply) |
| Shares/Reposts | `m.room.message` quoting original via reply |
| Threading | `m.thread` relationship |

**Reaction Example:**
```json
{
  "type": "m.reaction",
  "content": {
    "m.relates_to": {
      "rel_type": "m.annotation",
      "event_id": "$original_post_id",
      "key": "👍"
    }
  }
}
```

**Comment (Reply) Example:**
```json
{
  "type": "m.room.message",
  "content": {
    "msgtype": "m.text",
    "body": "Great post!",
    "m.relates_to": {
      "rel_type": "m.thread",
      "event_id": "$original_post_id",
      "is_falling_back": true,
      "m.in_reply_to": {
        "event_id": "$original_post_id"
      }
    }
  }
}
```

---

## 3. Feed Rooms & Privacy

### Room Types

| Room Purpose | Join Rules | Visibility |
|--------------|------------|------------|
| Public feed | `public` | `world_readable` |
| Friends-only feed | `restricted` | `shared` |
| Close friends | `invite` | `shared` |
| High-Privacy Broadcast | `invite` (Direct Messages) | `invited` |

### User's Feed Structure

Each user maintains **multiple feed rooms** for different audiences:

```
User's Social Space
├── #feed_alice:example.org (public posts)
├── #friends_alice:example.org (friends-only, restricted to friends space)
└── #close_alice:example.org (invite-only, close friends)
```

### Sharing Safeguards

**Strict Rule:** Clients MUST forbid or warn when sharing content from restricted rooms to public rooms.

| Action | Client Behavior |
|--------|-----------------|
| Share from friends-only to public | **BLOCK** or show confirmation dialog |
| Quote private content publicly | Display warning: "You are about to share private content publicly" |
| Forward mentions | Warn if mentioned users aren't in target room |

### Cross-Posting

Users post to multiple rooms simultaneously:
1. Post to public feed → visible to everyone
2. Post to friends feed → visible to friends only
3. Post to both → maximum reach with privacy layers

### Alternative: Hidden Friends List (Broadcast Mode)

If a user wishes to keep their friend list completely hidden (avoiding member list exposure), the architecture supports **Broadcast Mode**.

**Mechanism:**
1. **Storage**: Client maintains a private list of "Friends" in account data or locally
2. **Posting**: Client iterates through list and sends post as DM to each friend individually (using existing 1:1 encrypted rooms)
3. **Reception**: Friends see posts in their DM history with the sender

**Tradeoffs (User Warning Required):**

| Limitation | Impact |
|------------|--------|
| Bandwidth | Posting is O(n) requests instead of O(1) |
| Comment Fragmentation | Friend A's comment is only visible to poster, not Friend B |
| No Reaction Aggregation | Can't see "5 friends liked this" |
| No Social Proof | Lose "mutual friends who liked this" |
| Storage Duplication | Same post stored N times across homeservers |
| Loss of Control | Recipients can forward the DM publicly |
| No Central Feed | Posts live in DM history, not a browsable feed |

---

## 4. Newsfeed (Aggregated Timeline)

### Matrix Structures Used

| Feature | Matrix Primitive |
|---------|------------------|
| Real-time updates | `/sync` endpoint |
| Historical posts | `/messages` endpoint |
| Filtering | Sync filters |

### Implementation Strategy

**The newsfeed is the union of all joined feed rooms.**

```
Newsfeed = Sync(
  rooms: [
    friend1_public_feed,
    friend1_friends_feed,  // if you're their friend
    friend2_public_feed,
    friend3_public_feed,
    friend3_friends_feed,
    ...
  ]
)
```

### Performance & Caching

To handle potentially hundreds of feed rooms, clients MUST:

**1. Use Strict Sync Filters:**
```json
{
  "room": {
    "timeline": {
      "limit": 10,
      "types": ["m.room.message", "m.reaction", "m.room.redaction"]
    },
    "state": {
      "types": ["m.room.name", "m.room.avatar"],
      "lazy_load_members": true
    }
  },
  "presence": {
    "types": ["m.presence"]
  }
}
```

**2. Aggressive Local Caching:**
- Cache timeline events locally (IndexedDB for web, SQLite for native)
- Don't fetch remote history every session
- Implement incremental sync with `since` token

**3. Lazy Loading:**
- Always enable `lazy_load_members: true`
- Avoid downloading thousands of member state events
- Fetch member details on-demand when rendering

### Feed Algorithm (Client-Side)

Since Matrix delivers all events chronologically, the client can:

1. **Chronological**: Display as received (Twitter-style)
2. **Algorithmic**: Re-sort by engagement (reactions count)
3. **Grouped**: Cluster by user or topic

---

## 5. Friend Network (Social Graph)

### Matrix Structures Used

| Feature | Matrix Primitive |
|---------|------------------|
| Friends list | Space membership |
| Follow | Join someone's public feed room |
| Friend request | Knock on restricted room |
| Mutual friends | Bidirectional space membership |

### Friends Space Architecture

Each user maintains a **Friends Space** containing their network:

```json
{
  "preset": "private_chat",
  "name": "Alice's Friends",
  "room_alias_name": "friends_space_alice",
  "creation_content": {
    "type": "m.space"
  },
  "initial_state": [
    {
      "type": "m.room.join_rules",
      "content": { "join_rule": "invite" }
    }
  ]
}
```

> **Note:** For "Hidden Friends" (Broadcast Mode), this Space is used only for internal organization, or skipped entirely in favor of account data tagging.

### Friend Request Flow

```
1. Bob wants to friend Alice
   └── Bob knocks on Alice's friends-only feed room

2. Alice reviews request
   └── Alice sees knock event with Bob's profile

3. Alice accepts
   └── Alice invites Bob to her friends-only feed
   └── Alice joins Bob's public feed (follow back)
   └── Optional: Alice adds Bob to her friends space

4. Mutual friendship established
   └── Both can see each other's friends-only content
```

### Restricted Room Join Rules

```json
{
  "type": "m.room.join_rules",
  "content": {
    "join_rule": "restricted",
    "allow": [
      {
        "type": "m.room_membership",
        "room_id": "!friends_space:example.org"
      }
    ]
  }
}
```

---

## 6. Events (Gatherings)

### Matrix Structures Used

| Feature | Matrix Primitive |
|---------|------------------|
| Event details | Room state events |
| RSVPs | Custom state event per user |
| Discussion | Room messages |
| Attendee list | Room membership |
| Host permissions | `m.room.power_levels` |

### Permission Model (Strict Power Levels)

To ensure event integrity, strict Power Levels must be enforced upon room creation.

```json
{
  "type": "m.room.power_levels",
  "content": {
    "users_default": 0,
    "events_default": 0,
    "state_default": 50,
    "ban": 50,
    "kick": 50,
    "redact": 50,
    "invite": 0,
    "events": {
      "org.social.event": 50,
      "org.social.rsvp": 0,
      "m.room.name": 50,
      "m.room.avatar": 50
    },
    "users": {
      "@creator:example.org": 100
    }
  }
}
```

| Power Level | Role | Capabilities |
|-------------|------|--------------|
| 100 | Creator | Full control, can promote co-hosts |
| 50 | Co-Host/Moderator | Edit event details, moderate chat, kick/ban |
| 0 | Guest | Chat, RSVP, invite others (if allowed) |

> **Private Events:** For surprise parties or restricted events, set `"invite": 50` to prevent guests from inviting others.

### Event Room Structure

**Public Event:**
```json
{
  "preset": "public_chat",
  "name": "Summer BBQ 2026",
  "topic": "Annual neighborhood cookout",
  "room_alias_name": "bbq_2026",
  "initial_state": [
    {
      "type": "org.social.event",
      "state_key": "",
      "content": {
        "title": "Summer BBQ 2026",
        "description": "Join us for food, fun, and friends!",
        "start_time": 1751295600000,
        "end_time": 1751310000000,
        "location": {
          "name": "Central Park",
          "address": "123 Park Ave",
          "geo": "geo:40.7829,-73.9654"
        },
        "cover_image": "mxc://example.org/bbq_cover",
        "visibility": "public",
        "rsvp_deadline": 1751209200000
      }
    },
    {
      "type": "m.room.join_rules",
      "content": { "join_rule": "public" }
    }
  ]
}
```

**Private Event:**
```json
{
  "preset": "private_chat",
  "name": "Sarah's Surprise Birthday",
  "initial_state": [
    {
      "type": "org.social.event",
      "state_key": "",
      "content": {
        "title": "Sarah's Surprise Birthday",
        "description": "Shhh! It's a surprise!",
        "start_time": 1751382000000,
        "visibility": "private"
      }
    },
    {
      "type": "m.room.join_rules",
      "content": { "join_rule": "invite" }
    },
    {
      "type": "m.room.power_levels",
      "content": {
        "invite": 50
      }
    }
  ]
}
```

### RSVP System

Each user's RSVP is a state event with their user ID as `state_key`:

```json
{
  "type": "org.social.rsvp",
  "state_key": "@bob:example.org",
  "content": {
    "status": "going",
    "guests": 2,
    "note": "Bringing potato salad!"
  }
}
```

> **⚠️ Client Validation Required:** Matrix does NOT enforce that `state_key` matches `sender`. Clients MUST ignore `org.social.rsvp` events where `state_key` does not match the event's `sender` field to prevent impersonation.

### Event Discovery

| Type | Discovery Method |
|------|------------------|
| Public events | Public room directory, space hierarchy |
| Friends' events | Listed in friends' spaces |
| Invited events | Direct room invites |

### Event Features via Existing Primitives

| Feature | Implementation |
|---------|----------------|
| Event chat | Regular room messages |
| Photo sharing | `m.image` messages |
| Polls (what to bring) | `m.poll.start` events |
| Location sharing | `m.location` messages |
| Reminders | Push notification rules |

---

## 7. Complete User Space Hierarchy

```
@alice:example.org's Social Space
│
├── 📋 Profile Room (#profile_alice:example.org)
│   ├── [state] org.social.profile (bio, links, etc.)
│   ├── [state] m.room.avatar (profile picture)
│   └── [state] m.room.topic (tagline)
│
├── 📢 Public Feed (#feed_alice:example.org)
│   ├── [join_rule] public
│   ├── [history] world_readable
│   ├── [permissions] Only Alice can post (PL required)
│   └── [messages] public posts
│
├── 👥 Friends Feed (#friends_alice:example.org)
│   ├── [join_rule] restricted (to friends space)
│   ├── [history] shared
│   ├── [permissions] Only Alice can post
│   └── [messages] friends-only posts
│
├── ⭐ Close Friends (#close_alice:example.org)
│   ├── [join_rule] invite
│   ├── [history] shared
│   └── [messages] close friends posts
│
├── 👫 Friends Space (!friends_space_alice)
│   ├── [type] m.space
│   ├── [children] friend feed rooms Alice has access to
│   └── [join_rule] invite (Alice controls membership)
│
└── 📅 Events Space (!events_alice)
    ├── [type] m.space
    ├── [child] Summer BBQ Room (Public, PL 50 for hosts)
    └── [child] Birthday Party Room (Private, PL 50 for hosts)
```

---

## 8. Client Implementation Summary

### Key Endpoints Used

| Feature | Endpoint |
|---------|----------|
| Create room/space | `POST /createRoom` |
| Post content | `PUT /rooms/{id}/send/m.room.message/{txn}` |
| Get feed | `GET /sync` with filter |
| Upload media | `POST /media/v1/create` + `PUT` (async) |
| Get room hierarchy | `GET /rooms/{id}/hierarchy` |
| Search public rooms | `POST /publicRooms` |
| Get profile | `GET /profile/{userId}` |
| Set presence | `PUT /presence/{userId}/status` (if supported) |

### Media Handling (Async Upload)

Use the v1 authenticated media endpoints for reliable upload of large content:

```
1. POST /_matrix/media/v1/create → get mxc_uri and upload_url
2. PUT {upload_url} → upload file body
```

### Required Room Version

| Requirement | Version |
|-------------|---------|
| **Minimum** | Room Version 8 (for restricted join rules) |
| **Recommended** | Room Version 11 (current default in v1.17) |

### Custom Event Types (Namespaced)

| Type | Purpose |
|------|---------|
| `org.social.profile` | Extended profile data |
| `org.social.event` | Event/gathering details |
| `org.social.rsvp` | Event attendance |
| `org.social.link_preview` | Rich link previews |
| `org.social.caption` | Media captions |

---

## 9. Privacy Model Summary

| Content | Who Can See | Matrix Implementation |
|---------|-------------|----------------------|
| Public posts | Anyone | `world_readable` history, `public` join |
| Friends posts | Approved friends | `restricted` to friends space |
| Close friends | Hand-picked users | `invite` only |
| Hidden friends posts | Specific recipients | Direct Messages (Broadcast Mode) |
| Private events | Invited guests | `invite` only room |
| Public events | Anyone | `public` join, can peek |

---

## 10. Federation Benefits

Using Matrix protocol provides:

1. **No vendor lock-in** - Users can move to any Matrix homeserver
2. **Interoperability** - Works with any Matrix client
3. **Decentralization** - No single point of failure
4. **E2E encryption** - Optional encryption for sensitive content
5. **Existing infrastructure** - Leverage existing Matrix homeservers
6. **Identity portability** - 3PID linking for discovery

---

## 11. Implementation Phases

### Phase 1: Core Social
- Profile rooms with state events
- Public feed rooms
- Basic posting (text, images)
- Follow mechanism (join public feeds)

### Phase 2: Friend Network
- Friends space architecture
- Restricted feed rooms
- Friend request flow (knock/invite)
- Friends-only posts

### Phase 3: Rich Content
- Video posts
- Link previews
- Reactions and comments
- Threading

### Phase 4: Events
- Event rooms with metadata
- RSVP system
- Power level management
- Event discovery

### Phase 5: Privacy & Polish
- Broadcast mode for hidden friends
- Sharing safeguards
- Algorithmic feed sorting
- Rich notifications
- Search and discovery

---

## Conclusion

This architecture leverages 100% existing Matrix protocol features:

- **Rooms** → Content containers (feeds, profiles, events)
- **Spaces** → Organizational hierarchy and access control
- **Messages** → Posts, comments, media
- **State events** → Profile data, event details, RSVPs
- **Reactions** → Likes and engagement
- **Join rules** → Privacy levels (public, friends, private)
- **Power levels** → Host/moderator permissions
- **Sync API** → Real-time newsfeed
- **m.mentions** → v1.17 compliant user mentions

No server modifications needed. A standard Matrix homeserver (Synapse, Dendrite, Conduit) provides all required functionality.
