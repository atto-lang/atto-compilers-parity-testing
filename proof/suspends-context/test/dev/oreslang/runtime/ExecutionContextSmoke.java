package dev.oreslang.runtime;

public final class ExecutionContextSmoke {
    private static void check(boolean ok, String message) {
        if (!ok) throw new AssertionError(message);
    }
    private static void rejects(Runnable action) {
        try {
            action.run();
        } catch (IllegalArgumentException expected) {
            return;
        }
        throw new AssertionError("invalid parent identity unexpectedly admitted");
    }
    public static void main(String[] args) {
        ExecutionContext root = ExecutionContext.task(71, 1);
        ExecutionContext child = ExecutionContext.task(71, 2, root.id().execution());
        ExecutionContext grandchild = ExecutionContext.task(71, 3, child.id().execution());
        ExecutionContext hostRoot = ExecutionContext.task(72, 1);
        check(root.parentExecution() == 0, "root must not inherit a parent");
        check(child.parentExecution() == 1, "child must retain same-domain parent");
        check(grandchild.parentExecution() == 2, "grandchild must retain causal parent");
        check(root.id().runtimeDomain() == child.id().runtimeDomain(), "same scheduler domain");
        check(!root.id().equals(child.id()), "each task must have a unique identity");
        check(hostRoot.parentExecution() == 0, "host starts must be roots");
        check(child.allocationDomain().kind() == ExecutionContext.AllocationKind.NONE,
                "task lineage must never mint actor allocation authority");
        check(!child.allocationDomain().isBound(), "task must remain heap-unbound");
        check(child.continuation() == 0, "parent metadata must not forge a continuation");
        rejects(() -> ExecutionContext.task(71, 2, -1));
        rejects(() -> ExecutionContext.task(71, 2, 2));
        System.out.println("ATTO_EXECUTION_CONTEXT_HOST_PASS|"
                + root.id().runtimeDomain() + "|" + child.parentExecution());
    }
}
