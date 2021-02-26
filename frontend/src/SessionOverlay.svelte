<script>
  import QRCode from 'qrcode';
  import { SessionStatus } from './session';
  import ResetButton from './ResetButton.svelte';
  import { isMobile } from "./util";

  export let sessionState;
  export let cancel;

  let qrPromise = null;
  let link = null;
  let status;

  $: status = sessionState.status;
  $: if (sessionState.qrCode) {
    if (isMobile()) {
      link = `https://irma.app/-/session#${encodeURIComponent(sessionState.qrCode)}`;
    } else {
      qrPromise = QRCode.toDataURL(sessionState.qrCode, {
        type: 'svg',
        margin: 0,
        width: Math.min(window.innerHeight * 0.7 - 100, 500),
      });
    }
  }
</script>

<style lang="scss">
  .backdrop {
    position: fixed;
    top: 0;
    left: 0;
    bottom: 0;
    right: 0;
    background: rgba(#000, 0.5);

    .modal {
      position: absolute;
      top: 50%;
      left: 50%;
      transform: translate(-50%, -50%);
      background: #fff;
      padding: 1rem;
      font-size: 1.5rem;

      img {
        margin: 1rem;
      }

      .center {
        text-align: center;
        padding: 4rem;
      }
    }
  }
</style>

<div class="backdrop">
  <div class="modal {status}">
    {#if status === SessionStatus.INITIALIZED && qrPromise !== null}
      {#await qrPromise}
        <p>Starting your session.</p>
      {:then src}
        <p>Scan the QR code with the IRMA app.</p>
        <p>
          <img {src} alt="IRMA QR" />
        </p>
        <ResetButton cancel={cancel} title="Cancel" />
      {/await}
    {:else if status === SessionStatus.INITIALIZED && link}
      <p>Click the button below tho open the IRMA app.</p>
      <p class="center">
        <a href={link} target="_blank" class="button">
          Open IRMA
        </a>
      </p>
    {:else if status === SessionStatus.CONNECTED}
      <p>Follow the instructions in the IRMA app.</p>
      <ResetButton cancel={cancel} title="Cancel" />
    {:else if status === SessionStatus.CANCELLED}
      <p>The session was cancelled.</p>
      <ResetButton cancel={cancel} title="Close" />
    {:else if status === SessionStatus.TIMEOUT}
      <p>The session timed-out.</p>
      <ResetButton cancel={cancel} title="Close" />
    {:else if status === SessionStatus.DONE}
      <p>The verification was successful. You will be redirected.</p>
      <ResetButton cancel={cancel} title="Cancel" />
    {/if}
  </div>
</div>

