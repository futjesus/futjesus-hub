// Evaluated periodically inside each Gmail webview. Returns a JSON string
// with the inbox unread count, the signed-in address and its profile picture.
(function () {
  if (location.hostname !== 'mail.google.com') return JSON.stringify({});

  // The number Gmail shows next to "Inbox" in the left navigation. With inbox
  // tabs enabled it only counts the Primary tab, just like Gmail does. The row
  // is looked up explicitly because the Gmail logo also links to "#inbox".
  var unread = null;
  var rows = document.querySelectorAll('.aim');
  for (var i = 0; i < rows.length; i++) {
    if (rows[i].querySelector('a[href$="#inbox"]')) {
      var count = rows[i].querySelector('.bsU');
      unread = count ? parseInt(count.textContent.replace(/\D/g, ''), 10) || 0 : 0;
      break;
    }
  }
  if (unread === null && location.hash.indexOf('#inbox') === 0) {
    var m = document.title.match(/\((\d+)\)/);
    unread = m ? parseInt(m[1], 10) : 0;
  }

  var email = null;
  var fromTitle = document.title.match(/[^\s()]+@[^\s()]+\.[a-z]+/i);
  if (fromTitle) email = fromTitle[0];

  // The account button in Gmail's top bar holds the profile picture.
  var avatar = null;
  var img =
    document.querySelector('a[href*="accounts.google.com/SignOutOptions"] img') ||
    document.querySelector('header img[src*="googleusercontent.com"]');
  if (img && img.src) avatar = img.src.replace(/=s\d+(-[a-z0-9-]+)?$/i, '=s128-c');

  return JSON.stringify({ unread: unread, email: email, avatar: avatar });
})()
