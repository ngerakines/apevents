# apevents

***NOTICE: This project is under active development and is not ready for production.***

(ActivityPub) Events is an event management system that allows event creation and coordination across the fediverse.

Events are considered public and there is no current or planned implementation of "private" events. The server will support domain allow and deny lists, for both fediverse moderation as well as network controls, but if the site is public then events can be seen by practically anyone.

# Roadmap

* Event actors
  * Descriptions
  * Follow and unfollow actions
  * @event + #discuss public posts to boost messages to event followers
  * @event + #impersonate mention-only posts to have the event re-publish the message as the event without the #impersonate tag
  * @event + #admin mention-only post to have the event reply with an admin link that is good for 24 hours
* Public event page that includes event details
* Event creation with @planner
* Admin page for server configuration
  * Domain deny and allow functionality
* Media storage offloading to object storage

## Icebox / For Discussion

* Limited NLP functionality with the @planner actor to provide helpful functionality
* Fediverse identity verification
* Home page event and place lists
* Event actor and object expiration and cleanup

# Usage (abridged)

## Creating Events

1. Open up https://events.thegem.city/
2. Locate the form that says "create new event" and put in your actor reference (i.e. `@nick@thegem.city`, `https://thegem.city/@nick` or `https://thegem.city/users/nick`)
3. Check the direct messages of the created account and follow the link to https://events.thegem.city/events/readily-splendid-mule/admin?password=abcd1234
4. Fill in the event details:
   * Title - Actor display name
   * Description - Actor description
   * Starts at - Actor metadata "starts_at"
   * Ends at - Actor metadata "ends_at"
   * (optional) Location - Actor metadata "location"

Constraints:

*(Some of these are arbitrary.)*

* Event titles must be under 64 unicode characters.
* Event descriptions must be under 1000 unicode characters.
* The starts_at may not be more than 30 days in the past from now.
* The ends_at may not be more than 7 days from starts_at.
* Location must be under 300 characters.

## Updating events

1. Send a direct mesage to the event actor (i.e. `@readily-splendid-mule@events.thegem.city`) with the visibility "mentioned only" with the content "#admin" in the message
2. Check for a response from the event actor with a link to an admin page. That link is only good for a limited amount of time.
3. Edit any event details and configuration.

## Following an event

1. From your fediverse instance, search for the actor reference (i.e. `@readily-splendid-mule@events.thegem.city`, `https://events.thegem.city/@readily-splendid-mule` or `https://events.thegem.city/users/readily-splendid-mule`)
2. Follow the event actor

## RSVP

1. Send a direct message to the event actor with the message "#rsvp going" or "#rsvp not going"
2. Receive a confirmation direct message from the event actor.

# License

MIT License

Copyright (c) 2022 Nick Gerakines

See also: [LICENSE](./LICENSE)
