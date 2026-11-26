// A single specific four-line JDK 25 host warning for the pinned Truffle JAR.
// No general WARNING filter, arbitrary stderr suppression or JVM flag changes.
// The full original warning is retained in the case receipt.
export function classifyJvmStderr(stderr,home) {
  if(typeof home!=='string'||!home.startsWith('/')||home.includes('\n')||home.includes('\r'))return {hostWarning:null,guestStderr:stderr};
  const warning=[
    'WARNING: A terminally deprecated method in sun.misc.Unsafe has been called',
    'WARNING: sun.misc.Unsafe::objectFieldOffset has been called by com.oracle.truffle.api.nodes.NodeClassImpl$NodeFieldData (file:'+home+'/.m2/repository/org/graalvm/truffle/truffle-api/25.3.4.1/truffle-api-25.3.4.1.jar)',
    'WARNING: Please consider reporting this to the maintainers of class com.oracle.truffle.api.nodes.NodeClassImpl$NodeFieldData',
    'WARNING: sun.misc.Unsafe::objectFieldOffset will be removed in a future release'
  ].join('\n')+'\n';
  return stderr.startsWith(warning)?{hostWarning:warning,guestStderr:stderr.slice(warning.length)}:{hostWarning:null,guestStderr:stderr};
}
