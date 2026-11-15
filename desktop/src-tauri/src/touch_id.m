#import <LocalAuthentication/LocalAuthentication.h>
#import <dispatch/dispatch.h>

// Called on a blocking worker, never on the window event loop.
int cuetuck_touch_id(int prompt) {
    @autoreleasepool {
        LAContext *context = [[LAContext alloc] init];
        context.localizedFallbackTitle = @"";
        LAPolicy policy = LAPolicyDeviceOwnerAuthenticationWithBiometrics;
        if (![context canEvaluatePolicy:policy error:NULL]) return 0;
        if (!prompt) return 1;
        dispatch_semaphore_t done = dispatch_semaphore_create(0);
        __block BOOL verified = NO;
        [context evaluatePolicy:policy localizedReason:@"解锁 CueTuck" reply:^(BOOL success, NSError *error) {
            (void)error;
            verified = success;
            dispatch_semaphore_signal(done);
        }];
        dispatch_semaphore_wait(done, DISPATCH_TIME_FOREVER);
        return verified ? 1 : 0;
    }
}
