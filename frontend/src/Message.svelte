<script>
  export let message;
</script>

<style lang="scss">
  $primary: #00cc00;

  .header {
    display: flex;
    justify-content: space-between;
    color: #aaa;
    font-size: 0.8rem;

    strong {
      color: black;
      font-size: 0.9rem;
    }
  }

  .message {
    list-style: none;
    display: inline-block;
    min-width: 40%;
    max-width: 100%;
    background: white;
    padding: 0.5rem;
    margin: 0.5rem 1rem;
    width: auto;
    margin-right: auto;
    box-shadow: 0 3px 12px -6px rgba(0, 0, 0, 0.5);
    border-radius: 0.5rem;

    em {
      font-size: 0.9rem;
      color: #666;
    }

    &.me {
      background-color: lighten($primary, 58%);
      margin-left: auto;
      margin-right: 1rem;
    }
  }
</style>

<li class="message {message.its_me ? 'me' : 'not-me'}">
  {#if message.msg}
    <span class="header">
      <strong>{message.user}</strong>
      &nbsp;
      <span>{message.time}</span>
    </span>
    {message.msg}
  {:else}
    <em><strong>{message.user}</strong> joined on {message.time}</em>
  {/if}
</li>