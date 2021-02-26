<script>
  export let start;
  export let loading;
</script>

<style lang="scss">
  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  h2 {
    text-align: center;
    font-weight: bold;
  }

  button {
    &.start {
      display: block;
      font-weight: bold;
      width: 10rem;
      margin: 5rem auto;
      cursor: pointer;
    }

    &.loading {
      position: relative;
      color: rgba(#fff, 0);

      &::after{
        content: ' ';
        position: absolute;
        left: calc(50% - 0.75rem);
        top: 0.5rem;
        display: inline-block;
        width: 1.5rem;
        height: 1.5rem;
        border: 3px solid rgba(#fff, 0.4);
        border-radius: 50%;
        border-top-color: #fff;
        animation: spin 1s ease-in-out infinite;
      }
    }
  }
</style>

<h2>IRMA Chat</h2>
<button class="start" class:loading={loading} on:click={start}>
  Authenticate
</button>
