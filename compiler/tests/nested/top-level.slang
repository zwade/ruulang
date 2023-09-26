entity Company {
    read;

    member -> User;
}

fragment BasicData for Company {
    read;

    member {
        #BasicData;
    }
}

fragment BasicData for User {
    write.basic;
    read;

    company {
        #BasicData;
    }
}
/**
 * A user of a company
 */
@User {
    company {
        read;

        #BasicData;

        member {
            #BasicData;
        }
    }
}