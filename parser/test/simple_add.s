run:
	sub	sp, #96
	@APP
	sub	sp, #508
	@NO_APP
	@APP
	sub	sp, #452
	@NO_APP
	movs	r0, #1
	str	r0, [sp, #8]
	movs	r0, #2
	str	r0, [sp, #4]
	ldr	r0, [sp, #8]
	ldr	r1, [sp, #4]
	adds	r0, r0, r1
	str	r0, [sp]
	ldr	r0, [sp]
	str	r0, [sp, #36]
	b	.LBB0_1
.LBB0_1:
	b	.LBB0_2
.LBB0_2:
	b	.LBB0_2